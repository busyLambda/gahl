run:
	cargo b
	./target/debug/phalast b examples/main.gh

reset:
	rm out
	rm out.bs
	rm out.ll
	rm out.o

clean:
	make reset
	make run