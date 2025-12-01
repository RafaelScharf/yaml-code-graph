// In your src/main.ts file
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { ValidationPipe } from '@nestjs/common'; // Import the ValidationPipe

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // Apply the ValidationPipe globally
  app.useGlobalPipes(new ValidationPipe());

  await app.listen(3000);
}
bootstrap();