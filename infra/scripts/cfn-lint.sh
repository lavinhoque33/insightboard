#!/bin/sh
# Lint the synthesised templates with a pinned cfn-lint inside a throwaway
# python container (M6 decision 1). Nothing is installed on the host, the
# templates are mounted read-only, and no AWS credentials are involved.
# cfn-lint exits non-zero on errors and on warnings.
set -eu

CFN_LINT_VERSION=1.57.2
PYTHON_IMAGE=python:3.12-slim

cd "$(dirname "$0")/.."
if ! ls cdk.out/*.template.json >/dev/null 2>&1; then
	echo "cfn-lint: no cdk.out/*.template.json; run 'npm run synth' first" >&2
	exit 1
fi

exec docker run --rm \
	-v "$PWD/cdk.out:/templates:ro" \
	"$PYTHON_IMAGE" \
	sh -c "pip install --quiet --no-cache-dir --disable-pip-version-check --root-user-action=ignore cfn-lint==$CFN_LINT_VERSION \
		&& cfn-lint --version \
		&& cfn-lint /templates/*.template.json \
		&& echo 'cfn-lint: no errors or warnings'"
