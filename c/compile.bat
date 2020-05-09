@echo off
SET CompileFlags=-nologo -I3rdparty -Zi -Zo -W4 -Gm- -Fdbin/ -Febin/ -Fobin/ -D_CRT_SECURE_NO_WARNINGS -MT -O2
SET LinkOptions=-INCREMENTAL:NO -DEBUG
cl %CompileFlags% src/imgn.c -link %LinkOptions%
