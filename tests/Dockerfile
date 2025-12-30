FROM mcr.microsoft.com/playwright:v1.40.0-jammy

WORKDIR /workspace

COPY package*.json ./
RUN npm ci

COPY .  .

CMD ["npx", "playwright", "test"]