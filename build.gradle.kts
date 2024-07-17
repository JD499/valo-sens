plugins {
    id("java")
    id("org.graalvm.buildtools.native") version "0.10.2"
}

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

graalvmNative {
    binaries {
        named("main") {
            mainClass.set("com.github.jd499.valorant.pro.settings.scraper.Main")
            buildArgs.add("--enable-url-protocols=http")
            buildArgs.add("--enable-url-protocols=https")
        }
    }
}

tasks.test {
    useJUnitPlatform()
}