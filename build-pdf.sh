#!/bin/bash
(
    cd tex &&
    mkdir -p build &&
    echo "*" > build/.gitignore &&
    pdflatex -output-directory=build main.tex &&
    cp build/main.pdf ../Quantum\ Glossary.pdf
)
