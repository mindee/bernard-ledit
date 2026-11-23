<?php

declare(strict_types=1);

use BernardLedit\Pdf\PdfDocument;

if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    exit;
}

$bytes = file_get_contents('php://input');
if ($bytes === false) {
    throw new RuntimeException('Could not read the request body');
}

$document = new PdfDocument($bytes);
header('Content-Type: application/json');
echo json_encode(['size' => strlen($bytes), 'pages' => $document->getPageCount()], JSON_THROW_ON_ERROR);
