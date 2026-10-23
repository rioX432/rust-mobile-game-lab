import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "lab.rustmobile"
    compileSdk = 35
    buildToolsVersion = "35.0.0"
    ndkVersion = "28.2.13676358"
    System.getenv("ANDROID_NDK_HOME")?.let { ndkPath = it }
    defaultConfig {
        applicationId = "lab.rustmobile"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1"
        ndk { abiFilters += "arm64-v8a" }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
kotlin { compilerOptions { jvmTarget.set(JvmTarget.JVM_17) } }

dependencies {
    testImplementation("junit:junit:4.13.2")
}
