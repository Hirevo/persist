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

# stop the running process.
persist stop http-server

# restart the process.
persist restart http-server

# stop managing the process.
persist delete http-server

# stop the background deamon.
persist daemon kill
```
