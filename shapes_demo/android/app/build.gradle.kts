import java.util.Properties
import java.io.FileInputStream

plugins {
    id("com.android.application") version "8.9.2"
    id("org.mozilla.rust-android-gradle.rust-android") version "0.9.6"
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
    ndkVersion = "29.0.13113456 rc1"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.s2e_systems.dustddsshapesdemo"
        minSdk = 26
        targetSdk = 36
        versionCode = 6
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

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
}

apply(plugin = "org.mozilla.rust-android-gradle.rust-android")

cargo {
    module = "."
    targets = listOf("x86", "x86_64", "arm", "arm64")
    libname = "shapes_demo_app"
    verbose = true
    prebuiltToolchains = true
    targetDirectory = "../../../target"
    profile = "release"
}

dependencies {
    tasks.named("preBuild").configure {
        dependsOn("cargoBuild")
    }
}