Attempt at converting a normal RC car into an autonomous race car.

Uses: 

  - NVIDIA Jetson TX2 as the ECU
  - Adafruit PWM driver

Development Environmen:

I usually use CLion on Windows 10 and compile with clang. Perform the following steps to set it up:

  - Download and install the latest MinGW 64 bit from http://www.msys2.org/
  - Install clang from the msys shell: `pacman -S mingw-w64-x86_64-llvm mingw-w64-x86_64-clang`
  - (Optional) install the GNU Compiler Collection for good measure: `pacman -S make gcc gdb`
  - Open CLion and clone this project.
