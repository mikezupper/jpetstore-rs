# jpetstore-rs

A feature-parity port of [MyBatis JPetStore 6](https://github.com/mybatis/jpetstore-6)
to Rust: Axum, SQLx, Askama, SQLite. Same pet store, same features, one static
binary where the app server used to be.

This is the companion repository for the free course
**[Port a Classic Java App to Rust](https://mikezupper.com/courses/java-to-rust-jpetstore/)**
on mikezupper.com.

> **Status: the course is in production.** Code lands here lesson by lesson —
> watch or star the repo if you want to follow along as it's built.

## How this repo tracks the course

Each lesson gets a git tag (`lesson-01`, `lesson-02`, …) marking the state of
the code at the end of that lesson. `main` is always the latest completed
lesson. To put your own work next to lesson 5's checkpoint:

```sh
git checkout lesson-05
```

## Attribution

JPetStore 6 is © the [MyBatis team](https://github.com/mybatis), licensed
under Apache-2.0. This project is an independent port and keeps the same
license; see [LICENSE](LICENSE) and [NOTICE](NOTICE).
