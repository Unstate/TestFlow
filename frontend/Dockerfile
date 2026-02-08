# Stage 1: Build WASM
FROM eclipse-temurin:21-jdk AS builder

WORKDIR /app

# Copy Gradle wrapper and config files first (for caching)
COPY gradlew gradlew.bat ./
COPY gradle/ gradle/
COPY build.gradle.kts settings.gradle.kts gradle.properties ./
COPY composeApp/build.gradle.kts composeApp/build.gradle.kts
COPY .npmrc ./

# Make gradlew executable
RUN chmod +x gradlew

# Install libatomic (required by Kotlin's bundled Node.js for WASM)
RUN apt-get update && \
    apt-get install -y libatomic1 && \
    rm -rf /var/lib/apt/lists/*

# Copy source code
COPY composeApp/src/ composeApp/src/
COPY package.json ./

# Build WASM distribution
RUN ./gradlew :composeApp:wasmJsBrowserDistribution --no-daemon

# Stage 2: Serve with nginx
FROM nginx:alpine

COPY --from=builder /app/composeApp/build/dist/wasmJs/productionExecutable/ /usr/share/nginx/html/

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
