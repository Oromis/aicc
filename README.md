AICC (Artificial Intelligence Controlled Car)
=============================================

Attempt at converting a normal RC car into an autonomous race car.

Uses: 

  - NVIDIA Jetson TX2 as the ECU
  - Adafruit PWM driver
  
Project Plan
-----------

This project is in very early stages. I have a physical RC car to which I attached the Jetson board. At the moment, I'm busy with some hardware work (designing and building PCBs for sensor input, power supply and more). Things that work as of now:

  * I can control the steering servo and the electric motor from software on the VCU
  * I can connect to the Jetson Board (the ECU) via WiFi and remote-control the car using my keybard or an XBox gamepad
  
Roadmap:

  * Setup sensors: 
    * Accelerometer (located at the center of gravity of the board)
    * Battery Voltage monitoring (I'm fairly serious about electrical safety within my car)
    * Stereo cameras
    * A lidar (I haven't decided on a particular model yet)
  * Implement driving aids:
    * Anti-lock braking (ABS)
    * Traction control
    * Controlled drifting (with a set drifting angle)
  * Get the computer vision up and running
    * Be able to detect obstacles
    * Be able to detect the track markers
  * Start to train the car
    * The car shall learn the track layout by being driven manually for one or more laps
    * The car shall start to follow the track on its own, optimizing its line and discovering the limits of grip as it goes
    * Get faster and faster!

Architecture
------------

This project uses a microservice-based architecture. The entire functionality is split up into small programs that each have one responsibility. They communicate using TCP sockets, therefore it does not matter on which physical machine they are located. If I decide that logging shall be performed by a WiFi-connected box instead of the Jetson itself, this is just one change of a single variable away.

TCP sockets perform particularly well from localhost to localhost, where they are optimized to a simple memcpy() by modern Linuxes (actually, the reality ist much more complicated, but this analogy will do for this project).

The following Microservices exist at the moment:

  * drive-core: Handles the basic driving functions (throttle, braking, steering).
  * logging: Logs any data it receives to a very nice file format

Development Environment
----------------------

It's easiest to develop the code using Linux (any distribution will do, I use an Ubuntu 16.04 LTS).
CLion is an awesome IDE and it supports Rust, so I use it too. It's by no means necessary though!

  - Open CLion and clone this project.
  - Install rust (using rustup)
  - Install Jetson TX2 Cross Compiler (from https://developer.nvidia.com/embedded/downloads)
  - Install Rust cross compiler target: `rustup target add aarch64-unknown-linux-gnu`
  - Add the following to `~/.cargo/config`:
      ```
      [target.aarch64-unknown-linux-gnu]
      linker = "aarch64-unknown-linux-gnu-gcc"
      ```
  - Run `make` from the project root. The entire project should build.
