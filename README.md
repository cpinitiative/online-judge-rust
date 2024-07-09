To test base docker image (useful for determining what packages need to be installed)

```
docker run --rm -it --platform linux/amd64 --entrypoint /bin/bash public.ecr.aws/lambda/python:3.12
```

To build and test dockerfile locally

```
cargo lambda build
docker build --platform linux/amd64 -t oj-rust .
docker run --platform linux/amd64 -p 9000:8080 oj-rust

# or:
docker run -it --platform linux/amd64 -p 9000:8080 --entrypoint /bin/bash oj-rust

# to send command:
curl "http://localhost:9000/2015-03-31/functions/function/invocations" -d '{}'
```

To upload to ECR

```
# Login to AWS ECR
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 417600709600.dkr.ecr.us-east-1.amazonaws.com

# Tag the image
docker tag oj-rust 417600709600.dkr.ecr.us-east-1.amazonaws.com/online-judge-rust:latest

# Push
docker push 417600709600.dkr.ecr.us-east-1.amazonaws.com/online-judge-rust:latest
```

To deploy lambda for the first time:
- Create lambda through AWS console
- Add a function URL with CORS
- Set timeout to 15 seconds
- Set memory to 1769 MB (1 vCPU)

Future updates: Maybe https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lambda/update-function-code.html ?

---

Todo:
- precompile `bits/stdc++.h`

---


```js
for (let i = 0; i < 100; i++) fetch("https://v3nuswv3poqzw6giv37wmrt6su0krxvt.lambda-url.us-east-1.on.aws/compile", {
     method: "POST", 
    headers: {"Content-Type": "application/json" }, body: JSON.stringify({
    "source_code": "cat /proc/cpuinfo && sleep 1",
    "compiler_options": "-O2 -std=c++17",
    "language": "cpp"
}) }).then(x => x.json()).then(x => console.log(x.compile_output.stdout.match(/cpu MHz\t\t: (.*)/)[1]))
```