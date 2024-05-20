FROM osrf/ros:noetic-desktop-full

RUN apt-get update && apt-get install -y \
    curl build-essential gfortran python3-catkin ros-noetic-catkin python3-catkin-tools wget \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /home/catkin_ws/src

# Install Rust using the official installation script
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add Rust to the PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify installation
RUN rustc --version

COPY . /home/catkin_ws/src/termviz

RUN cd /home/catkin_ws/src/termviz && . "/opt/ros/${ROS_DISTRO}/setup.sh" && RUST_BACKTRACE=1 cargo build

# Download example bag file
