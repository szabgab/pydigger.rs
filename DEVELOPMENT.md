# Development



```
git clone https://github.com/szabgab/pydigger.rs
git clone https://github.com/szabgab/pydigger-front
cd pydigger.rs
ln -s ../pydigger-fron/html/index.html
```


Collect data and generated report json file

```
cargo run -- --download --report
```

## View the web site locally

* Install [rustatic](https://rustatic.code-maven.com/) and run

```
rustatic --path . --indexfile index.html
```


