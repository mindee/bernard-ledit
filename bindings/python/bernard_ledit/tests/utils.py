import os
from pathlib import Path

OUTPUT_DIR = Path(__file__).resolve().parents[4] / "tests" / "data" / "output"

def cleanup_output_files(created_files):
    for file_path in created_files:
        full_path = OUTPUT_DIR / file_path
        if full_path.exists():
            os.remove(full_path)
