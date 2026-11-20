vcpkg_from_github(
    OUT_SOURCE_PATH SOURCE_PATH
    REPO strukturag/libheif
    REF "v${VERSION}"
    SHA512 556cc0e365aacad662d8c20282b68856b51af168659acd0f2662dcb399e5fed9dea9c259f5d5ffcd383f4b4a00a93acbdc06e90b340cc1dd2098e94c30c8a606
)

if("libde265" IN_LIST FEATURES)
    set(with_libde265 ON)
    set(with_libde265_plugin ON)
else()
    set(with_libde265 OFF)
    set(with_libde265_plugin OFF)
endif()

vcpkg_cmake_configure(
    SOURCE_PATH "${SOURCE_PATH}"
    OPTIONS
        -DBUILD_TESTING=OFF
        -DBUILD_DOCUMENTATION=OFF
        -DENABLE_PLUGIN_LOADING=ON
        -DPLUGIN_DIRECTORY=
        -DWITH_EXAMPLES=OFF
        -DWITH_GDK_PIXBUF=OFF
        -DWITH_LIBSHARPYUV=OFF
        "-DWITH_LIBDE265=${with_libde265}"
        "-DWITH_LIBDE265_PLUGIN=${with_libde265_plugin}"
        -DWITH_FFMPEG_DECODER=OFF
        -DWITH_X265=OFF
        -DWITH_X264=OFF
        -DWITH_OpenH264_DECODER=OFF
        -DWITH_AOM_DECODER=OFF
        -DWITH_AOM_ENCODER=OFF
        -DWITH_DAV1D=OFF
        -DWITH_RAV1E=OFF
        -DWITH_SvtEnc=OFF
        -DWITH_KVAZAAR=OFF
        -DWITH_JPEG_DECODER=OFF
        -DWITH_JPEG_ENCODER=OFF
        -DWITH_OpenJPEG_DECODER=OFF
        -DWITH_OpenJPEG_ENCODER=OFF
        -DWITH_OPENJPH_ENCODER=OFF
        -DWITH_UVG266=OFF
        -DWITH_VVDEC=OFF
        -DWITH_VVENC=OFF
    OPTIONS_RELEASE
        "-DPLUGIN_INSTALL_DIRECTORY=${CURRENT_PACKAGES_DIR}/plugins/libheif"
    OPTIONS_DEBUG
        "-DPLUGIN_INSTALL_DIRECTORY=${CURRENT_PACKAGES_DIR}/debug/plugins/libheif"
)
vcpkg_cmake_install()
vcpkg_copy_pdbs()
vcpkg_cmake_config_fixup(CONFIG_PATH "lib/cmake/libheif")
vcpkg_fixup_pkgconfig()
file(REMOVE_RECURSE "${CURRENT_PACKAGES_DIR}/debug/include" "${CURRENT_PACKAGES_DIR}/debug/share")
file(GLOB plugins "${CURRENT_PACKAGES_DIR}/plugins/libheif/*")
if(plugins STREQUAL "")
    file(REMOVE_RECURSE "${CURRENT_PACKAGES_DIR}/plugins" "${CURRENT_PACKAGES_DIR}/debug/plugins")
endif()
vcpkg_install_copyright(FILE_LIST "${SOURCE_PATH}/COPYING")
