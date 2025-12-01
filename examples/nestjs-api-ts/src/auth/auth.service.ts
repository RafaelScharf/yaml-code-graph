import { Injectable } from '@nestjs/common';
import { UsersService } from '../users/users.service';

@Injectable()
export class AuthService {
  /**
   * @param usersService O serviço para interagir com dados do usuário.
   */
  constructor(private usersService: UsersService) {}

  /**
   * Valida um usuário com base nas credenciais.
   * @param username O nome de usuário.
   * @param pass A senha do usuário.
   * @returns O objeto do usuário, exceto a senha, se a validação for bem-sucedida.
   */
  async validateUser(username: string, pass: string): Promise<any> {
    try {
        const user = await this.usersService.findOne(username);
        
        // Lógica simulada de validação de senha
        if (user && user.password === pass) {
          const { password, ...result } = user;
          return result;
        }
    } catch (e) {
        // Se findOne lançar exceção (usuário não encontrado)
        return null;
    }
    
    return null;
  }
}
