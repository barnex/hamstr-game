#! /bin/bash

set -e
size=64

for f in *.svg; do
	g=../$(echo $f | sed 's/.svg/.png/g');
	if [ $f -nt $g ]; then
		echo inkscape -z -w $size $f -e $g;
		inkscape -z -w $size $f -e $g;
	fi;
done

for f in *.png; do
	g=../$f;
	if [ $f -nt $g ]; then
		echo convert -scale $size $f $g;
		convert -scale $size $f $g;
	fi;
done