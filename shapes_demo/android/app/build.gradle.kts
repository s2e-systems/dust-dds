import java.util.Properties
import java.io.FileInputStream

plugins {
    id("com.android.application") version "8.13.0"
}

android {
    signingConfigs {
        create("release") {
            val keystorePropertiesFile = rootProject.file("keystore.properties")
            val keystoreProperties = Properties()
            if (keystorePropertiesFile.exists()) {
                keystoreProperties.load(FileInputStream(keystorePropertiesFile))

                keyAlias = keystoreProperties["keyAlias"] as String
                keyPassword = keystoreProperties["keyPassword"] as String
                storeFile = file(keystoreProperties["storeFile"] as String)
                storePassword = keystoreProperties["storePassword"] as String
            }
        }
    }
    namespace = "com.s2e_systems.dustddsshapesdemo"
    ndkVersion = "29.0.14033849"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.s2e_systems.dustddsshapesdemo"
        minSdk = 26
        versionCode = 7
        versionName = "1.0.13"
    }

    buildTypes {
        getByName("release") {
            isDebuggable = false
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("release")
        }
    }
    sourceSets["main"].jniLibs.srcDirs(layout.buildDirectory.dir("rustJniLibs"))
}

val cargoBuild by tasks.registering(Exec::class) {
    val outputDir = layout.buildDirectory.dir("rustJniLibs")
    // The --platform should be "android.compileSdk", but the ndk did not include a prebuilt
    // for API level (compileSdk) 36
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

