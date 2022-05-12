SRCS = $(wildcard samples/*.c)
PROGS = $(patsubst %.c,%,$(SRCS))

all: $(PROGS)

%: %.c
	$(CC) $(CFLAGS) -O0 -g -no-pie -fno-omit-frame-pointer -o $@ $<

clean:
	rm -f $(PROGS)

run:
	docker exec -it rust 'cd /usr/src/deet && cargo run'

docker:
	docker run --name rust  -v /Users/zjc/CLionProjects/CS110L/proj-1:/usr/src/ -it arm64v8/rust:1.60.0