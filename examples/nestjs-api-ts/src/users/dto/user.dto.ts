// Define a estrutura de dados de retorno do usuário
export interface UserDto {
  /**
   * O ID exclusivo do usuário.
   */
  userId: number;
  /**
   * O nome de usuário único.
   */
  username: string;
}

// Define a estrutura completa do usuário (inclui campos sensíveis para uso interno)
export interface InternalUser {
  /**
   * O ID exclusivo do usuário.
   */
  userId: number;
  /**
   * O nome de usuário único.
   */
  username: string;
  /**
   * A senha do usuário (em um ambiente real, seria hashed).
   */
  password: string;
}
