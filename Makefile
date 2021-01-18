all: build publish

build:
	wasm-pack build --scope repeatnotes

publish:
	jq '.files |= . + ["sm2_bg.js"]' pkg/package.json > pkg/package2.json \
		&& mv pkg/package{2,}.json \
		&& wasm-pack publish --access public --tag next
