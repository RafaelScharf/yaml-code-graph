import { Injectable } from '@nestjs/common';
import { InternalUser } from './dto/user.dto';

// Banco de dados simulado
const mockUsers: InternalUser[] = [
  { userId: 1, username: 'rafael', password: '123' },
  { userId: 2, username: 'julia', password: '456' },
];

@Injectable()
export class UsersService {
  /**
   * Busca um usuário pelo nome de usuário.
   * @param username O nome de usuário para buscar.
   * @returns O objeto InternalUser encontrado ou undefined.
   */
  async findOne(username: string): Promise<InternalUser | undefined> {
    const user = mockUsers.find(u => u.username === username);
    // Lança exceção se o usuário não for encontrado para fins de exemplo
    if (!user) {
        throw new Error('Usuário não encontrado');
    }
    return user;
  }

  /**
   * Busca um usuário pelo ID.
   * @param id O ID do usuário para buscar.
   * @returns O objeto InternalUser encontrado ou undefined.
   */
  async findById(id: number): Promise<InternalUser | undefined> {
    const user = mockUsers.find(u => u.userId === id);
    // Lança exceção se o usuário não for encontrado para fins de exemplo
    if (!user) {
        throw new Error('Usuário não encontrado');
    }
    return user;
  }
}
