To run normally:

```
cargo lambda watch
``` 

And POST `http://localhost:9000/compile-and-execute`.

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
- Set timeout to 30 seconds
- Set memory to 1769 MB (1 vCPU)


---

file stuff: relaxed permissions

---

Todo: Note that there are actually two CPUs -- AMD and Intel -- so timings are not consistent. With this, I saw 1.93 and also 1.30:

```
double x; cin >> x; int y = x; for (int i = 0; i < y; i++) x += sqrt(x); cout << x << endl;
```

x = 200000000 input.

```js
for (let i = 0; i < 100; i++) fetch("https://v3nuswv3poqzw6giv37wmrt6su0krxvt.lambda-url.us-east-1.on.aws/compile-and-execute", {
     method: "POST", 
    headers: {"Content-Type": "application/json" }, body: JSON.stringify({compile:{
    "source_code": `// Source: https://usaco.guide/general/io

#include <bits/stdc++.h>
using namespace std;

int main() {
double x; cin >> x; int y = x; for (int i = 0; i < y; i++) x += sqrt(x); cout << x << endl;;
}

`,
    "compiler_options": "-O2 -std=c++17",
    "language": "cpp"
},execute:{stdin:"200000000", timeout_ms:5000}}) }).then(x => x.json()).then(x => console.log(x.execute.stderr.match(/wall clock.*/)[0]))
```

Should benchmark this to determine how off the timings are / whether we can just add a multiplicative factor to it.

---

misc todos

```
// timeout: warning: disabling core dumps failed: Operation not permitted
// Command exited with non-zero status 137
// UGH

// also, internal server error when output too large

// want smth like
/*
Line 15: Char 8: runtime error: signed integer overflow: 2147483647 + 2147483647 cannot be represented in type 'int' (solution.cpp)
SUMMARY: UndefinedBehaviorSanitizer: undefined-behavior solution.cpp:15:8
*/
// which is what leetcode gives you
```
