plugins {
    id("com.android.application") version "8.13.0"
}

android {
    signingConfigs {
        create("release") {
            keyAlias = "key1"
            keyPassword = System.getenv("ANDROID_RELEASE_KEY_PASSWORD")
            storeFile = file("keystore.jks")
            storePassword = System.getenv("ANDROID_RELEASE_STORE_PASSWORD")
        }
    }
    namespace = "com.s2e_systems.dustddsshapesdemo"
    ndkVersion = "29.0.14033849"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.s2e_systems.dustddsshapesdemo"
        minSdk = 26
        targetSdk = 36
        versionCode = 8
        versionName = "1.0.13"
    }

    buildTypes {
        getByName("release") {
            isDebuggable = false
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
            )
            signingConfig = signingConfigs.getByName("release")
        }
    }
    sourceSets["main"].jniLibs.srcDirs(layout.buildDirectory.dir("rustJniLibs"))
}

val cargoBuild by tasks.registering(Exec::class) {
    val outputDir = layout.buildDirectory.dir("rustJniLibs")
    commandLine(
        "cargo", "ndk",
        "--target", "arm64-v8a",
        "--target", "armeabi-v7a",
        "--target", "x86_64",
        "--target", "x86",
        "--platform", 35,
        "--output-dir", outputDir.get().asFile.absolutePath,
        "build", "--release",
        "--package", "shapes_demo_app"
    )
}

tasks.named("preBuild") {
    dependsOn(cargoBuild)
}

