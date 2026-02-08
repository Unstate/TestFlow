package ru.foxteam.testflow

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform