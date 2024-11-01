# wsl-drive-automount

Tool to automatically mount Windows drives in WSL2 when connected.

## Usage

```bash
# In wsl
$ sudo mkdir /mnt/e # Create a mount point for the drive

# In Windows
$ git clone https://github.com/nazo6/wsl-drive-automount
$ cargo run --release
```

After running the program, drives will be automatically mounted to
`/mnt/<drive letter>` when connected.
