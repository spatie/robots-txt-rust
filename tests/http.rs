//! Real loopback HTTP requests and local-file loading. No external server is required.

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use spatie_robots_txt::{Error, Robots, RobotsHeaders, RobotsMeta, RobotsTxt};

struct Server {
    origin: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    fn new(robots_status: u16) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => panic!("{error}"),
                };
                // Windows accepts inherit the listener's non-blocking mode.
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                        break;
                    }
                    request.push_str(&line);
                }
                if request.is_empty() {
                    continue;
                }
                let path = request
                    .lines()
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .nth(1)
                    .unwrap();
                let (status, headers, body) = match path {
                    "/robots.txt" => (
                        robots_status,
                        "",
                        "User-agent: *\nDisallow: /blocked\nUser-agent: google\nDisallow: /private\n",
                    ),
                    "/meta" => (200, "", "<meta name=\"robots\" content=\"noindex\">"),
                    "/follow" => (200, "", "<meta name=\"robots\" content=\"nofollow\">"),
                    "/headers" => (200, "X-Robots-Tag: nofollow\r\n", "<html></html>"),
                    "/none" => (200, "X-Robots-Tag: none\r\n", "<html></html>"),
                    "/scoped" => (200, "X-Robots-Tag: google: none\r\n", "<html></html>"),
                    "/repeated" => (
                        200,
                        "X-Robots-Tag: noindex\r\nX-Robots-Tag: nofollow\r\n",
                        "<html></html>",
                    ),
                    "/redirect" => (302, "Location: /none\r\n", ""),
                    "/missing" => (404, "", "not found"),
                    _ => (200, "", "<html></html>"),
                };
                seen.lock().unwrap().push(request);
                write!(stream, "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}", body.len()).unwrap();
            }
        });
        Self {
            origin,
            requests,
            stop,
            handle: Some(handle),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.origin)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(self.origin.trim_start_matches("http://"));
        self.handle.take().unwrap().join().unwrap();
    }
}

#[test]
fn discovers_origin_rules_and_short_circuits_denied_pages() {
    let server = Server::new(200);
    let robots = Robots::create();
    assert!(
        !robots
            .may_index(&server.url("/blocked?x=1#section"))
            .unwrap()
    );
    assert!(robots.may_index(&server.url("/open?x=1")).unwrap());
    let requests = server.requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /robots.txt HTTP/1.1\r\n"));
    assert!(requests[2].starts_with("GET /open?x=1 HTTP/1.1\r\n"));
}

#[test]
fn combines_meta_and_headers_but_follow_does_not_consult_robots_txt() {
    let server = Server::new(200);
    let robots = Robots::create().with_txt(RobotsTxt::new("User-agent: *\nDisallow: /blocked"));
    assert!(!robots.may_index(&server.url("/meta")).unwrap());
    assert!(robots.may_follow_on(&server.url("/meta")).unwrap());
    assert!(robots.may_index(&server.url("/follow")).unwrap());
    assert!(!robots.may_follow_on(&server.url("/follow")).unwrap());
    assert!(robots.may_index(&server.url("/headers")).unwrap());
    assert!(!robots.may_follow_on(&server.url("/headers")).unwrap());
    assert!(!robots.may_index(&server.url("/none")).unwrap());
    assert!(!robots.may_follow_on(&server.url("/none")).unwrap());
    assert!(robots.may_follow_on(&server.url("/blocked")).unwrap());
}

#[test]
fn keeps_http_user_agent_separate_from_rule_override_and_global_headers() {
    let server = Server::new(200);
    let robots = Robots::create().user_agent("UserAgent007");
    assert!(
        !robots
            .may_index_for(&server.url("/private"), "google")
            .unwrap()
    );
    assert!(robots.may_index(&server.url("/scoped")).unwrap());
    assert!(robots.may_follow_on(&server.url("/scoped")).unwrap());
    let headers = RobotsHeaders::read_from(&server.url("/scoped")).unwrap();
    assert!(!headers.may_index_for("google"));
    assert!(!headers.may_follow_for("google"));
    let requests = server.requests.lock().unwrap();
    assert!(
        requests
            .iter()
            .filter(|r| r.starts_with("GET /robots.txt "))
            .all(|r| r.contains("user-agent: spatie/robots-txt\r\n"))
    );
    assert!(
        requests
            .iter()
            .filter(|r| r.starts_with("GET /scoped "))
            .take(2)
            .all(|r| r.contains("user-agent: UserAgent007\r\n"))
    );
}

#[test]
fn loads_response_headers_repeats_redirects_and_meta() {
    let server = Server::new(200);
    let headers = RobotsHeaders::read_from(&server.url("/repeated")).unwrap();
    assert!(headers.may_index());
    assert!(!headers.may_follow());
    assert!(
        RobotsMeta::read_from(&server.url("/meta").replacen("http:", "HTTP:", 1))
            .unwrap()
            .noindex()
    );
    assert!(
        !RobotsHeaders::read_from(&server.url("/redirect"))
            .unwrap()
            .may_index()
    );
    assert!(
        RobotsMeta::read_from(&server.url("/meta"))
            .unwrap()
            .noindex()
    );
    assert!(
        !RobotsTxt::try_read_from(&server.url("/robots.txt"))
            .unwrap()
            .allows("/blocked")
    );
    assert!(
        !Robots::create()
            .with_txt_from(&server.url("/robots.txt"))
            .may_index(&server.url("/blocked"))
            .unwrap()
    );
}

#[test]
fn missing_robots_fails_open_but_missing_pages_return_errors() {
    let server = Server::new(404);
    assert!(Robots::create().may_index(&server.url("/open")).unwrap());
    assert!(RobotsTxt::read_from(&server.url("/robots.txt")).allows("/blocked"));
    assert!(matches!(
        RobotsTxt::try_read_from(&server.url("/robots.txt")),
        Err(Error::Http(_))
    ));
    assert!(matches!(
        Robots::create().may_index(&server.url("/missing")),
        Err(Error::Http(_))
    ));
    assert!(RobotsMeta::read_from(&server.url("/missing")).is_err());
    assert!(RobotsHeaders::read_from(&server.url("/missing")).is_err());
}

#[test]
fn local_sources_and_unreadable_source_policies() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/noindex.html");
    assert!(RobotsMeta::read_from(path).unwrap().noindex());
    assert!(
        !Robots::create()
            .with_txt(RobotsTxt::new(""))
            .may_index(path)
            .unwrap()
    );
    assert!(Robots::create().may_follow_on(path).unwrap());
    assert!(RobotsHeaders::read_from(path).unwrap().may_index());
    let txt = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/robots.txt");
    assert!(!RobotsTxt::try_read_from(txt).unwrap().allows("/en/admin/"));
    let missing = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/missing-source");
    assert!(RobotsTxt::read_from(missing).allows("/anything"));
    assert!(matches!(
        RobotsTxt::try_read_from(missing),
        Err(Error::Io(_))
    ));
    assert!(RobotsMeta::read_from(missing).is_err());
    assert!(RobotsHeaders::read_from(missing).is_err());
    assert!(Robots::create().may_index("not a url").is_err());
    let empty_agent = RobotsTxt::new("User-agent: *\nAllow: /\nUser-agent:\nDisallow: /blocked");
    assert!(
        !Robots::create()
            .with_txt(empty_agent)
            .may_index("/blocked")
            .unwrap()
    );
}

#[test]
fn toggles_restore_default_rule_selection_and_pairs_accept_repeated_headers() {
    let txt = RobotsTxt::new(
        "User-agent: *\nDisallow: /global\nCrawl-delay: 1.5\nUser-agent: bot\nDisallow: /bot",
    );
    let txt = txt
        .ignore_global_group()
        .with_partial_matches()
        .include_global_group()
        .exact_matches_only();
    assert!(!txt.allows("/global"));
    assert!(txt.allows_for("/bot", "Browser bot/1.0"));
    assert_eq!(txt.crawl_delay(), Some("1.5"));
    assert!(txt.content().contains("Crawl-delay: 1.5"));
    let headers =
        RobotsHeaders::from_pairs([("X-Robots-Tag", "noindex"), ("X-Robots-Tag", "nofollow")]);
    assert!(headers.may_index());
    assert!(!headers.may_follow());
}
