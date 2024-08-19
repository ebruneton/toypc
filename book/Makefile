SVG_FIGURES := $(shell find . -name "*.svg")
CV_CIRCUITS := $(shell find . -name "*.cv" -not -name "T8.cv")
TEX_FIGURES := $(SVG_FIGURES:%.svg=%.tex) $(CV_CIRCUITS:%.cv=%.tex)
T8 := part1/figures/chapter5

all: figures
	cargo run -p preprocessor
	latexmk -pdf -pdflatex="pdflatex --shell-escape %O %S" -outdir=build -auxdir=build -emulate-aux-dir toypc.tex

figures: $(TEX_FIGURES) $(T8)/T8-left.tex

%.tex: %.svg src/svg2tikz.py
	python3 src/svg2tikz.py $< $@

%.tex: %.cv src/cv2tikz.py
	python3 src/cv2tikz.py $< $@

%/T8-left.tex: %/T8.cv src/cv2tikz.py
	python3 src/cv2tikz.py $< $(@:%-left.tex=%-right.tex) -800 5000
	python3 src/cv2tikz.py $< $@ -5000 -800

clean:
	rm -f $(TEX_FIGURES) $(T8)/T8-left.tex $(T8)/T8-right.tex
	find . -name "*_generator.rs" -exec rm {} +
	find . -type d -name "generated" -exec rm -rf {} +
	rm -rf website
	rm -rf build
