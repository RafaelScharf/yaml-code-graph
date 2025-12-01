import { Controller, Post, Body } from '@nestjs/common';
import { AuthService } from './auth.service';

@Controller('auth')
export class AuthController {
  constructor(private authService: AuthService) {}

  /**
   * Endpoint de login. Tenta validar o usuário.
   * @param loginDto Objeto contendo credenciais (username, password).
   * @returns Um token de acesso simulado e dados do usuário.
   */
  @Post('login')
  async login(@Body() loginDto: { username: string, password: string }) {
    const user = await this.authService.validateUser(loginDto.username, loginDto.password);

    if (user) {
        return { 
            access_token: 'jwt-simulado-abc12345',
            user: user
        };
    }
    return { message: 'Credenciais inválidas' };
  }
}
