@echo off
SET CommonCompileFlags=-nologo -I3rdparty -Zi -Zo -W4 -Gm- -Fdbin/ -Febin/ -Fobin/ -D_CRT_SECURE_NO_WARNINGS -DNDEBUG -MT -O2
SET CommonLinkOptions=-INCREMENTAL:NO -DEBUG -OPT:REF -OPT:ICF

SET ImgnCompileFlags=
cl %CommonCompileFlags% %ImgnCompileFlags% src/imgn.c -link %CommonLinkOptions%

SET TabsenseCompileFlags=
SET TabsenseLinkFlags=
cl %CommonCompileFlags% %TabsenseCompileFlags% src/tabsense.c -link %CommonLinkOptions% %TabsenseLinkFlags%
