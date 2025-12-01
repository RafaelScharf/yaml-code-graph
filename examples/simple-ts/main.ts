/**
 * Calcula uma saudação personalizada.
 * @param name O nome do usuário.
 */
function hello(name: string) {
    // GUARD CLAUSE (Logic Lifting deve pegar isso)
    if (name.length === 0) {
        throw new Error("Name cannot be empty");
    }

    return 'Hello ' + name;
}

class User {
    id: number;
    constructor(id: number) {
        // Validação (Logic Lifting deve pegar isso)
        if (id < 0) {
            return;
        }
        this.id = id;
    }
}

function main() {
    const u = new User(1);
    console.log(hello("Dev"));
}