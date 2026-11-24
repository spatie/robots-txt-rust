<?php

// Run with PHP 8.1+: php tests/generate-fixtures.php ../robots-txt > tests/fixtures/php.json
// Reference: spatie/robots-txt df0f6c8cdaa6565d9413db30b4a321a15cbd529e
$source = $argv[1] ?? throw new InvalidArgumentException('Pass the PHP package checkout path.');
spl_autoload_register(function (string $class) use ($source): void {
    $file = $source.'/src/'.substr($class, strlen('Spatie\\Robots\\')).'.php';
    if (is_file($file)) {
        require $file;
    }
});

$contents = [];
foreach (glob($source.'/tests/data/*.txt') as $file) {
    $contents[basename($file)] = file_get_contents($file);
}
$contents += [
    'weighted wildcards' => "User-agent: *\nDisallow: /policies/\nAllow: /*/shipping-policy\nAllow: */policies/terms-of-service\n",
    'anchored allow' => "User-agent: *\nDisallow: /files/\nAllow: /files/*.html$\n",
    'longer disallow' => "User-agent: *\nAllow: /shop/\nDisallow: /shop/*/checkout\n",
    'ties' => "User-agent: *\nDisallow: /abc\nAllow: /abc\nDisallow: /def/\nAllow: /de*/\n",
    'reasons' => "User-agent: *\nDisallow: /hello\nDisallow: /hello-world\nDisallow: /goodbye\nUser-agent: google\nDisallow: /hello\nDisallow: /hello-world\nDisallow: /hello-kitty\n",
    'repeat group' => "User-agent: *\nDisallow: /hello\nUser-agent: google\nDisallow: /hello\nUser-agent: google\nAllow: /hello-world\n",
    'empty and fallback' => "User-agent: *\nDisallow: /\nAllow: /hello-world\nCrawl-delay: 1.5\nUser-agent: google\nDisallow:\n",
    'literal regexp and unicode' => "User-agent: *\nDisallow: /a.b[0](x)+\nDisallow: /café\nAllow: /café/menu\nDisallow: /*?\nAllow: /safe?\n",
    'empty group' => "User-agent: *\nDisallow: /hello\nUser-agent: google\n",
    'directives without paths' => "User-agent: *\nDisallow:\nAllow:\nCrawl-delay:\nUser-agent: google\nCrawl-delay: 5\n",
    'comments and spacing' => "# robots\n User-Agent: *\r\n\r\n Disallow: /hello # note\r\n Allow: /safe\r\n",
    'delay group boundaries' => "User-agent: google\nUser-agent: bing\nCrawl-delay: 1.5\nUser-agent: *\nCrAwL-dElAy: 10\nUser-agent: google\nDisallow: /hello\n",
    'wildcard edge cases' => "User-agent: *\nDisallow: /a**b*c$\nDisallow: /\$\nDisallow: /file$\nAllow: /a*b*allowed$\n",
    'unknown directives' => "User-agent: *\nSitemap: https://example.com/map.xml\nHost: example.com\nDisallow: /hello\n",
];
$paths = [
    '/', '/en/admin', '/en/admin/', '/en/admin/users', '/en/admin?print=true',
    '/fr/admin', '/fr/admin/', '/fr/admin?', '/fr/admin?#fragment', '/fr/admin?test',
    '/es/admin-disallow/', '/Case-Sensitive/Disallow', '/case-sensitive/disallow', '/no-agents',
    '/abc/1234/explicit.html', '/abc/1234/not_mentioned.html', '/xyz/1234/explicit.html',
    '/xyz/1234', '/nb/reindrift/', '/sitemap.xml', '/some_random/sub/site',
    '/policies/shipping-policy', '/policies/terms-of-service', '/policies/refund-policy',
    '/files/report.html', '/files/report.html?download=1', '/files/report.pdf',
    '/shop/shoes', '/shop/shoes/checkout', '/abcd', '/def/page', '/hello', '/hello-world',
    '/goodbye', '/safe?', '/café', '/café/menu', '/a.b[0](x)+', '/axb0x',
    '/aZZbYYc', '/file', '/file\n', 'https://example.com/hello-world?x=1#fragment',
    'https://example.com/fr/admin?', '//example.com/en/admin?print',
    '/hello # note', 'https://example.com/map.xml', 'lay: 1.5', '',
];
$agents = ['*', 'google', ' GOOGLE ', 'bing', 'UserAgent007', 'UserAgent010', 'UserAgent011', 'UserAgent012', 'UserAgent013', 'only-me', 'Mozilla/5.0 (compatible; UserAgent007/1.1)'];
$txt = [];
foreach ($contents as $name => $content) {
    $modes = [];
    foreach ([false, true] as $partial) {
        foreach ([false, true] as $ignoreGlobal) {
            $robots = new Spatie\Robots\RobotsTxt($content);
            if ($partial) {
                $robots->withPartialMatches();
            }
            if ($ignoreGlobal) {
                $robots->ignoreGlobalGroup();
            }
            $checks = [];
            foreach ($agents as $agent) {
                foreach ($paths as $path) {
                    $checks[] = [
                        'path' => $path,
                        'agent' => $agent,
                        'allows' => $robots->allows($path, $agent),
                        'reasons' => $robots->whyDisallows($path, $agent),
                    ];
                }
            }
            $delays = [];
            foreach ($agents as $agent) {
                $delays[$agent] = $robots->crawlDelay($agent);
            }
            $modes[] = compact('partial', 'ignoreGlobal', 'checks', 'delays');
        }
    }
    $txt[] = compact('name', 'content', 'modes');
}

$htmls = [];
foreach (glob($source.'/tests/data/*.html') as $file) {
    $htmls[] = file_get_contents($file);
}
$htmls = array_merge($htmls, [
    '<html></html>', '<meta name="robots" content="NONE">',
    '<META NAME=\'ROBOTS\' CONTENT=\'NOINDEX, NOFOLLOW\'>',
    "<meta name=\"robots\"\ncontent=\"noindex\">",
    '<meta content="noindex" name="robots">',
    '<meta name="google" content="noindex">',
    '<meta name="robots" content="follow"><meta name="robots" content="noindex">',
    '<meta name="robots" content="noindex"><meta name="robots" content="follow">',
    '<meta name="robots" data-note="noindex" content="follow">',
    '<meta name="robots" content="NOINDEX,nofollow"/>',
]);
$meta = [];
foreach ($htmls as $html) {
    $robots = Spatie\Robots\RobotsMeta::create($html);
    $meta[] = ['html' => $html, 'result' => [$robots->mayIndex(), $robots->mayFollow(), $robots->noindex(), $robots->nofollow()]];
}

$headers = [];
$headerSets = [[], ['X-custom: test']];
foreach (['', 'noindex', 'nofollow', 'none', 'NOINDEX, NOFOLLOW', 'index, follow', 'unavailable_after: 25 Jun 2020 15:00:00 PST', 'xnoindex'] as $options) {
    foreach (['', 'google: ', 'Google: '] as $prefix) {
        $headerSets[] = ['X-custom: test', 'x-RoBoTs-TaG: '.$prefix.$options];
    }
}
$headerSets = array_merge($headerSets, [
    ['X-Robots-Tag: nofollow', 'X-Robots-Tag: google: noindex, nofollow'],
    ['X-Robots-Tag: none', 'X-Robots-Tag: google: index, follow'],
    ['X-Robots-Tag: noindex', 'X-Robots-Tag: nofollow'],
    ['X-Robots-Tag: google: noindex', 'X-Robots-Tag: google: nofollow'],
]);
foreach ($headerSets as $lines) {
    $robots = Spatie\Robots\RobotsHeaders::create($lines);
    foreach (['*', 'google', 'Google', 'other-bot'] as $agent) {
        $headers[] = ['lines' => $lines, 'agent' => $agent, 'result' => [
            $robots->mayIndex($agent), $robots->mayFollow($agent),
            $robots->noindex($agent), $robots->nofollow($agent), $robots->none($agent),
        ]];
    }
}

echo json_encode([
    'reference' => 'df0f6c8cdaa6565d9413db30b4a321a15cbd529e',
    'txt' => $txt, 'meta' => $meta, 'headers' => $headers,
], JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE | JSON_THROW_ON_ERROR)."\n";
