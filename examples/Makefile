buildclibs:
	rm clibs/core.so
	clang -shared -o clibs/core.so clibs/core.c

cleanclibs:
	rm clibs/core.so

cleanbuild:
	rm out.ll
	rm out.o
	rm out

run:
	../target/debug/phalast b main.gh
	./out
