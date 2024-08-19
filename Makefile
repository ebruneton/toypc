all: book-all webapp-all public-all

clean:
	$(MAKE) -C book clean
	cargo clean
	rm -rf public

book-all:
	$(MAKE) -C book

webapp-all:
	$(MAKE) -C webapp

public-all: public/webapp.wasm public/scripts.zip public/sources.zip public/toys.zip
	cp -R website/* public/
	cp -R book/website/backups/ public/
	cp -R book/website/part2/ public/
	cp -R book/website/part3/ public/
	cp book/cover/front.jpg public/figures/
	cp book/build/toypc.pdf public/

# wasm-opt can be installed with binaryen
public/webapp.wasm: target/wasm32-unknown-unknown/web-release/webapp.wasm
	mkdir -p public
	wasm-opt -all -O3 -o public/webapp.wasm target/wasm32-unknown-unknown/web-release/webapp.wasm

public/scripts.zip: $(wildcard book/website/backups/*.txt) \
 $(wildcard book/website/part2/*.txt) \
 $(wildcard book/website/part3/*.txt) \
 $(wildcard scripts/src/*.py) \
 scripts/src/LICENSE \
 scripts/README.html
	mkdir -p public
	rm -f $@
	cd book/website && zip ../../$@ backups/*.txt part2/*.txt part3/*.txt
	cd scripts/src && zip ../../$@ *.py LICENSE
	cd scripts && zip ../$@ README.html
	
public/sources.zip: $(wildcard book/website/sources/*.txt) book/src/LICENSE
	mkdir -p public
	rm -f $@
	cd book/website/sources && zip ../../../$@ *.txt
	cd book/src && zip ../../$@ LICENSE
	
public/toys.zip: $(wildcard book/website/toys/*) \
 book/src/LICENSE
	mkdir -p public
	rm -f $@
	cd book/website/toys && zip -r ../../../$@ *
	cd book/src && zip ../../$@ LICENSE
	
