set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR arm)

set(CMAKE_SYSROOT /opt/fsl-imx-x11/4.1.15-1.2.0/sysroots/cortexa9hf-vfp-neon-poky-linux-gnueabi)

set(tools /opt/fsl-imx-x11/4.1.15-1.2.0/sysroots/x86_64-pokysdk-linux/)
set(CMAKE_C_COMPILER ${tools}/usr/bin/arm-poky-linux-gnueabi-gcc)
set(CMAKE_CXX_COMPILER ${tools}/usr/bin/arm-poky-linux-gnueabi-g++)

set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
