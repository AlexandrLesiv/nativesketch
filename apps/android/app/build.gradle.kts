plugins { id("com.android.application") }
android {
    namespace = "dev.nativesketch"
    compileSdk = 37
    defaultConfig { applicationId = "dev.nativesketch"; minSdk = 26; targetSdk = 37; versionCode = 1; versionName = "0.1"; ndk { abiFilters += "arm64-v8a" } }
}
dependencies { implementation("com.google.android.material:material:1.13.0") }
