"""
$ rust-objcopy -O ihex target/thumbv7em-none-eabihf/release/leadscrew-teensy leadscrew-teensy.hex
$ teensy_loader_cli --mcu=TEENSY41 leadscrew-teensy.hex
$ cat /dev/ttyACM0
"""

for the cat to work, check the group of that file and add user to it.
For arch thats uucp

to find which tty, try $ dmesg | grep tty
