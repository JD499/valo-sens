plugins {
    id("java")
    id("com.github.johnrengelman.shadow") version "8.1.1"
    id("org.beryx.runtime") version "1.12.7"
}

group = "com.github.jd499.valorant.pro.settings.scraper"



repositories {
    mavenCentral()
}

dependencies {
    // https://mvnrepository.com/artifact/org.jsoup/jsoup
    implementation("org.jsoup:jsoup:1.17.2")
    // https://mvnrepository.com/artifact/org.apache.commons/commons-math3
    implementation("org.apache.commons:commons-math3:3.6.1")

}

dependencies {
    testImplementation(platform("org.junit:junit-bom:5.9.1"))
    testImplementation("org.junit.jupiter:junit-jupiter")
}

project.setProperty("mainClassName", "com.github.jd499.valorant.pro.settings.scraper.Main")

tasks.test {
    useJUnitPlatform()
}

tasks.jar {
    manifest {
        attributes["Main-Class"] = "com.github.jd499.valorant.pro.settings.scraper.Main"
    }
}

runtime {
    options.set(listOf("--strip-debug", "--no-header-files", "--no-man-pages"))
    modules = listOf("java.desktop", "java.xml", "jdk.crypto.ec")


    jpackage {
        installerType = "msi"
        installerName = "ValorantProSettingsScraper"
        installerOptions = listOf("--win-console", "--win-menu", "--win-shortcut")
        imageOptions = listOf("--win-console")


    }

}





