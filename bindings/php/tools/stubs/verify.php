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

if (($argv[1] ?? '') === 'snapshot') {
    $extension = new ReflectionExtension('bernard_ledit');
    $api = ['functions' => [], 'classes' => []];
    foreach ($extension->getFunctions() as $function) {
        $api['functions'][$function->getName()] = signature($function);
    }
    foreach ($extension->getClasses() as $class) {
        $methods = [];
        foreach ($class->getMethods(ReflectionMethod::IS_PUBLIC) as $method) {
            if ($method->getDeclaringClass()->getName() === $class->getName()) {
                $methods[$method->getName()] = signature($method);
            }
        }
        $api['classes'][$class->getName()] = $methods;
    }
    if (!$api['functions'] || !$api['classes']) {
        throw new RuntimeException('Extension has no registered functions/classes; refusing empty stubs.');
    }
    echo json_encode($api, JSON_THROW_ON_ERROR);
} elseif (($argv[1] ?? '') === 'check' && count($argv) === 4) {
    $expected = json_decode(file_get_contents($argv[3]), true, flags: JSON_THROW_ON_ERROR);
    require $argv[2];
    foreach ($expected['functions'] as $name => $wanted) {
        if (!function_exists($name) || signature(new ReflectionFunction($name)) !== $wanted) {
            throw new RuntimeException("Stub signature differs from extension: $name");
        }
    }
    foreach ($expected['classes'] as $name => $methods) {
        if (!class_exists($name, false)) {
            throw new RuntimeException("Stub class missing: $name");
        }
        foreach ($methods as $method => $wanted) {
            if (!method_exists($name, $method) || signature(new ReflectionMethod($name, $method)) !== $wanted) {
                throw new RuntimeException("Stub signature differs from extension: $name::$method");
            }
        }
    }
    echo "Stub API matches the loaded extension.\n";
} else {
    throw new InvalidArgumentException('Usage: verify.php snapshot | check STUB API_JSON');
}
