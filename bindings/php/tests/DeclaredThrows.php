<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use PHPUnit\Framework\Assert;
use Throwable;

/**
 * Checks thrown exceptions against the `@throws` tags of bernard_ledit.stubs.php, so a class the
 * binding throws but the stubs don't declare fails the suite.
 */
trait DeclaredThrows
{
    /** @var array<string, list<string>>|null Declared classes, by lower-cased `function` or `Class::method`. */
    private static ?array $declaredThrows = null;

    /**
     * Asserts that `$call` throws exactly `$class`, that the stubs declare it for `$symbol`
     * (`Namespace\function` or `Namespace\Class::method`), and that the message contains `$message`.
     */
    protected static function assertThrowsDeclared(
        string $symbol,
        string $class,
        callable $call,
        ?string $message = null
    ): void {
        try {
            $call();
        } catch (Throwable $e) {
            Assert::assertSame($class, $e::class, "$symbol threw: {$e->getMessage()}");
            Assert::assertContains(
                $class,
                self::declaredThrows($symbol),
                "$symbol throws $class, but the stubs don't declare it"
            );
            if ($message !== null) {
                Assert::assertStringContainsString($message, $e->getMessage());
            }
            return;
        }
        Assert::fail("$symbol didn't throw $class");
    }

    /**
     * Returns the classes the stubs declare for `$symbol`, without the leading backslash.
     *
     * @return list<string>
     */
    protected static function declaredThrows(string $symbol): array
    {
        self::$declaredThrows ??= self::parseStubs(__DIR__ . '/../bernard_ledit.stubs.php');
        $key = strtolower($symbol);
        Assert::assertArrayHasKey($key, self::$declaredThrows, "$symbol isn't in the stubs");
        return self::$declaredThrows[$key];
    }

    /**
     * The stubs can't be loaded next to the extension, so they're read with the tokenizer.
     *
     * @return array<string, list<string>>
     */
    private static function parseStubs(string $path): array
    {
        $declared = [];
        $namespace = '';
        $class = null;
        $classDepth = 0;
        $depth = 0;
        $doc = null;
        $tokens = \PhpToken::tokenize(file_get_contents($path));
        foreach ($tokens as $i => $token) {
            if ($token->is(T_DOC_COMMENT)) {
                $doc = $token->text;
            } elseif ($token->is(T_NAMESPACE)) {
                $name = self::nextToken($tokens, $i);
                $namespace = $name->is([T_STRING, T_NAME_QUALIFIED]) ? $name->text . '\\' : '';
            } elseif ($token->is(T_CLASS)) {
                $class = self::nextToken($tokens, $i)->text;
                $classDepth = $depth + 1;
            } elseif ($token->is(T_FUNCTION)) {
                $name = self::nextToken($tokens, $i)->text;
                $symbol = $namespace . ($class === null ? $name : "$class::$name");
                preg_match_all('/@throws\s+\\\\?(\S+)/', $doc ?? '', $matches);
                $declared[strtolower($symbol)] = $matches[1];
                $doc = null;
            } elseif ($token->text === '{') {
                $depth++;
            } elseif ($token->text === '}') {
                if ($depth-- === $classDepth) {
                    $class = null;
                }
            }
        }
        return $declared;
    }

    /**
     * @param list<\PhpToken> $tokens
     */
    private static function nextToken(array $tokens, int $i): \PhpToken
    {
        while ($tokens[++$i]->isIgnorable()) {
        }
        return $tokens[$i];
    }
}
