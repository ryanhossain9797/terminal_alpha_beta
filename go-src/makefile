lib-arm:
	env GOOS=linux GOARCH=arm CGO_ENABLED=1 CC=arm-linux-gnueabihf-gcc go build -buildmode=c-archive -o ../golibs/libpeople.a main.go
lib:
	go build -buildmode=c-archive -o ../golibs/libpeople.a main.go	