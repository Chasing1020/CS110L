CSRCS = $(wildcard samples/*.c)
CPROGS = $(patsubst %.c,%,$(CSRCS))
GOSRCS = $(wildcard samples/*.go)
GOPROGS = $(patsubst %.go,%,$(GOSRCS))

all: $(CPROGS) $(GOPROGS)

%: %.c
	$(CC) $(CFLAGS) -O0 -g -no-pie -fno-omit-frame-pointer -o $@ $<

%: %.go
	CGO_LDFLAGS=-no-pie CGO_ENABLED=1 go build -ldflags=-w -gcflags=all="-N -l" -o $@ $<

clean:
	rm -f $(CPROGS) $(GOPROGS)

run:
	docker exec -it rust 'cd /usr/src/deet && cargo run'

docker:
	docker run --name rust  -v /Users/zjc/CLionProjects/CS110L/proj-1:/usr/src/ -it arm64v8/rust:1.60.0