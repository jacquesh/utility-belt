@echo off
SET CompileFlags=-nologo -I3rdparty -Zi -Zo -W4 -Gm- -EHa- -Fdbin/ -Febin/ -Fobin/ -D_CRT_SECURE_NO_WARNINGS -MT -O2
SET LinkOptions=-INCREMENTAL:NO -DEBUG
cl %CompileFlags% src/vssgen.cpp -link %LinkOptions% ole32.lib advapi32.lib oleAut32.lib
