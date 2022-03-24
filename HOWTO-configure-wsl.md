For the multicast communication to work on Linux, the following command must be
run:
```
# route add -net 224.0.0.0 netmask 240.0.0.0 dev lo
```

This is not persistent accross reboots. On Ubuntu, it can be made persistent
with Netplan[^1]; however I did not manage to make it work on
WSL[^2]. Additionally, Netplan doesn't work on every distribution;
some use a config file in `/etc/sysconfig` (which doesn't exist on Ubuntu).

Another solution is to run the above command at startup. On a native Linux
installation this can be done with a systemd service, but WSL doesn't have
systemd. On Windows 11 there is a way to run a command at boot[^3] but it's not
available on Windows 10, so in the end I just added the following to
`~/.bashrc`:

```bash
if [ -z "$(ip route show dev lo)" ]
then
    # `wsl.exe -u root` is used to run as root without using sudo[^3]
    wsl.exe -u root route add -net 224.0.0.0 netmask 224.0.0.0 dev lo
fi
```


[^1]: 
https://www.ubuntu-server.com/tutorials/adding-persistent-static-routes-on-ubuntu-18-04-and-higher-using-netplan/

[^2]: The output of `netplan --debug apply` contains the following:
```
DEBUG:no netplan generated networkd configuration exists
DEBUG:no netplan generated NM configuration exists
```

[^3]: https://askubuntu.com/a/1356147