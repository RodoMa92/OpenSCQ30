import org.jlleitschuh.gradle.ktlint.KtlintExtension

// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    id("com.android.application") version "8.5.0" apply false
    id("com.android.library") version "8.5.0" apply false
    id("org.jetbrains.kotlin.android") version "2.0.0" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.0" apply false
    id("com.google.dagger.hilt.android") version "2.51.1" apply false
    id("com.google.devtools.ksp") version "2.0.0-1.0.22" apply false
    id("org.jlleitschuh.gradle.ktlint") version "12.1.0"
    id("com.google.protobuf") version "0.9.4" apply false
    kotlin("plugin.serialization") version "2.0.0"
}

configure<KtlintExtension> {
    android.set(true)
}
