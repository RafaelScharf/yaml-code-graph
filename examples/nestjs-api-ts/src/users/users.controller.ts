import { Controller, Get, Param, ParseIntPipe } from '@nestjs/common';
import { UsersService } from './users.service';
import { UserDto } from './dto/user.dto';

@Controller('users')
export class UsersController {
  constructor(private usersService: UsersService) {}

  /**
   * Endpoint de exemplo para buscar um usuário por ID.
   * @param id O ID do usuário (deve ser um número inteiro).
   * @returns Os dados públicos do usuário (UserDto).
   */
  @Get(':id')
  async findOne(@Param('id', ParseIntPipe) id: number): Promise<UserDto> {
    const user = await this.usersService.findById(id);

    if (user) {
        return { 
            userId: user.userId, 
            username: user.username 
        };
    }
    
    throw new Error('Usuário não encontrado');
  }
}
