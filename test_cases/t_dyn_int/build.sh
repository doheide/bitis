#!/bin/bash
# Builds and runs this test case with the shared runner (see ../run_case.sh).
exec "$(dirname "$0")/../run_case.sh" "$(dirname "$0")"
