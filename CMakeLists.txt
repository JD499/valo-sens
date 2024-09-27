cmake_minimum_required(VERSION 3.28)
project(valorantProSettingsScraper C)

set(CMAKE_C_STANDARD 17)

find_package(CURL REQUIRED)



find_package(LibXml2 REQUIRED)




add_executable(ValorantProSettingsScraper main.c)




target_link_libraries(ValorantProSettingsScraper PRIVATE LibXml2::LibXml2
        CURL::libcurl)
