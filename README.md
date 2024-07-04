To build and test dockerfile locally

```
docker build --platform linux/amd64 -t oj-rust .
docker run --platform linux/amd64 -p 9000:8080 oj-rust

# or:
docker run -it --platform linux/amd64 -p 9000:8080 --entrypoint /bin/bash oj-rust

# to send command:
curl "http://localhost:9000/2015-03-31/functions/function/invocations" -d '{}'
```
