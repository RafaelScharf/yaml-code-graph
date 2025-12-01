import { Injectable, CanActivate, ExecutionContext } from '@nestjs/common';
import { Observable } from 'rxjs';

@Injectable()
export class AuthGuard implements CanActivate {
  /**
   * Simula a validação de um token de acesso na requisição.
   */
  canActivate(
    context: ExecutionContext,
  ): boolean | Promise<boolean> | Observable<boolean> {
    const request = context.switchToHttp().getRequest();
    
    // Simula a verificação de um cabeçalho 'Authorization'
    const token = request.headers['authorization'];
    
    if (token && token.startsWith('Bearer ')) {
        // Lógica real de validação de JWT seria aqui.
        // Retorna true para acesso permitido
        return true;
    }

    // Retorna false para acesso negado
    return false;
  }
}
