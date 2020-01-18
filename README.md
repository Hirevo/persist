persist
=======

persist is a fast and simple asynchronous process manager.

Example
-------

```bash
# start managing a new process.
persist start --name http-server -- serve .

# list managed processes.
persist ls
persist list

# stop the running process.
persist stop http-server

# dump the current process configurations.
persist dump --all

# restart the process.
persist restart http-server

# stop managing the process.
persist delete http-server

# restore the process from the previous dump
persist restore --all

# stop managing all processes.
persist delete --all

# stop the background deamon.
persist daemon kill
```

License
-------

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
