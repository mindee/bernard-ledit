<?php
declare(strict_types=1);

set_error_handler(static function (int $severity, string $message, string $file, int $line): never {
    throw new ErrorException($message, 0, $severity, $file, $line);
});

function signature(ReflectionFunctionAbstract $function): array
{
    return [
        'return' => (string) $function->getReturnType(),
        'static' => $function instanceof ReflectionMethod && $function->isStatic(),
        'parameters' => array_map(static fn (ReflectionParameter $parameter): array => [
            'name' => $parameter->getName(),
            'type' => (string) $parameter->getType(),
            'nullable' => $parameter->allowsNull(),
            'optional' => $parameter->isOptional(),
            'variadic' => $parameter->isVariadic(),
            'reference' => $parameter->isPassedByReference(),
        ], $function->getParameters()),
    ];
}

/**
 * Public methods declared on the class itself.
 */
function publicMethods(ReflectionClass $class): array
{
    $methods = [];
    foreach ($class->getMethods(ReflectionMethod::IS_PUBLIC) as $method) {
        if ($method->getDeclaringClass()->getName() === $class->getName()) {
            $methods[$method->getName()] = signature($method);
        }
    }
    return $methods;
}

function declaredTypes(): array
{
    return array_merge(get_declared_classes(), get_declared_interfaces(), get_declared_traits());
}

if (($argv[1] ?? '') === 'snapshot') {
    $extension = new ReflectionExtension('bernard_ledit');
    $api = ['functions' => [], 'classes' => []];
    foreach ($extension->getFunctions() as $function) {
        $api['functions'][$function->getName()] = signature($function);
    }
    foreach ($extension->getClasses() as $class) {
        $api['classes'][$class->getName()] = publicMethods($class);
    }
    if (!$api['functions'] || !$api['classes']) {
        throw new RuntimeException('Extension has no registered functions/classes; refusing empty stubs.');
    }
    echo json_encode($api, JSON_THROW_ON_ERROR);
} elseif (($argv[1] ?? '') === 'check' && count($argv) === 4) {
    $expected = json_decode(file_get_contents($argv[3]), true, flags: JSON_THROW_ON_ERROR);

    $functionsBefore = get_defined_functions()['user'];
    $typesBefore = declaredTypes();
    require $argv[2];

    $actual = ['functions' => [], 'classes' => []];
    foreach (array_diff(get_defined_functions()['user'], $functionsBefore) as $lowercased) {
        $function = new ReflectionFunction($lowercased);
        $actual['functions'][$function->getName()] = signature($function);
    }
    foreach (array_diff(declaredTypes(), $typesBefore) as $name) {
        $actual['classes'][$name] = publicMethods(new ReflectionClass($name));
    }

    $errors = [];

    foreach (array_diff_key($expected['functions'], $actual['functions']) as $name => $_) {
        $errors[] = "Stub is missing function: $name";
    }
    foreach (array_diff_key($actual['functions'], $expected['functions']) as $name => $_) {
        $errors[] = "Stub has extra function not in extension: $name";
    }
    foreach (array_diff_key($expected['classes'], $actual['classes']) as $name => $_) {
        $errors[] = "Stub is missing class: $name";
    }
    foreach (array_diff_key($actual['classes'], $expected['classes']) as $name => $_) {
        $errors[] = "Stub has extra class not in extension: $name";
    }

    // Function signatures.
    foreach ($expected['functions'] as $name => $wanted) {
        if (isset($actual['functions'][$name]) && $actual['functions'][$name] !== $wanted) {
            $errors[] = "Stub signature differs from extension: $name";
        }
    }

    foreach ($expected['classes'] as $name => $methods) {
        if (!isset($actual['classes'][$name])) {
            continue;
        }
        foreach (array_diff_key($methods, $actual['classes'][$name]) as $method => $_) {
            $errors[] = "Stub is missing method: $name::$method";
        }
        foreach (array_diff_key($actual['classes'][$name], $methods) as $method => $_) {
            $errors[] = "Stub has extra method not in extension: $name::$method";
        }
        foreach ($methods as $method => $wanted) {
            if (isset($actual['classes'][$name][$method]) && $actual['classes'][$name][$method] !== $wanted) {
                $errors[] = "Stub signature differs from extension: $name::$method";
            }
        }
    }

    if ($errors) {
        throw new RuntimeException(implode("\n", $errors));
    }
    echo "Stub API matches the loaded extension.\n";
} else {
    throw new InvalidArgumentException('Usage: verify.php snapshot | check STUB API_JSON');
}