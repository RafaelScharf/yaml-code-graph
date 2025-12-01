# Logic Keywords Reference

This document provides a comprehensive reference for the logic keywords used in YCG's Level 2 (Inline Logic) granularity output. These keywords represent common programming patterns in a compact, semantic format optimized for LLM analysis.

## Table of Contents

- [Overview](#overview)
- [Keyword Reference](#keyword-reference)
  - [check(condition)](#checkcondition)
  - [action(operation)](#actionoperation)
  - [return(expression)](#returnexpression)
  - [match(pattern)?true:false](#matchpatterntrue-false)
  - [get(source)](#getsource)
- [Logic Chaining](#logic-chaining)
- [Complex Examples](#complex-examples)
- [Extraction Rules](#extraction-rules)
- [Edge Cases](#edge-cases)

## Overview

Logic keywords provide a compact representation of method implementation logic, focusing on:

1. **Control Flow:** Conditional checks and branching
2. **Side Effects:** State mutations and operations
3. **Data Flow:** Return values and data retrieval
4. **Business Logic:** Decision making and pattern matching

### Design Principles

- **Semantic Focus:** Capture intent, not syntax
- **Token Efficiency:** Short keywords, minimal punctuation
- **Deterministic:** Same code always produces same logic representation
- **Readable:** Human-readable without referring to source code
- **Chainable:** Multiple steps connected with semicolons

## Keyword Reference

### `check(condition)`

**Purpose:** Represents conditional checks, guard clauses, and preconditions

**Semantic Meaning:** "This condition must be true for execution to continue"

#### Source Patterns

##### 1. Guard Clause with Throw

```typescript
if (!user.isActive) {
  throw new Error("User is not active");
}
```

**Logic:** `check(user.isActive)`

**Note:** Condition is negated because the check represents what must be true.

##### 2. Guard Clause with Early Return

```typescript
if (balance < amount) {
  return false;
}
```

**Logic:** `check(balance >= amount)`

**Note:** Condition is inverted to represent the positive requirement.

##### 3. Simple If Statement

```typescript
if (user.role === 'admin') {
  // ... admin logic
}
```

**Logic:** `check(user.role === 'admin')`

##### 4. Complex Conditions with Logical Operators

```typescript
if (user.isActive && (user.isAdmin || user.isSuperUser)) {
  // ... authorized logic
}
```

**Logic:** `check(user.isActive && (user.isAdmin || user.isSuperUser))`

**Note:** Logical operators (`&&`, `||`, `!`) are preserved.

##### 5. Null/Undefined Checks

```typescript
if (!user) {
  throw new Error("User not found");
}
```

**Logic:** `check(user)`

```typescript
if (data === null || data === undefined) {
  return;
}
```

**Logic:** `check(data)`

##### 6. Multiple Conditions (Chained)

```typescript
if (stock <= 0) {
  throw new Error("Out of stock");
}
if (user.balance < price) {
  throw new Error("Insufficient funds");
}
```

**Logic:** `check(stock > 0);check(user.balance >= price)`

**Note:** Each check becomes a separate logic step.

#### When to Use

- Validation logic
- Preconditions
- Guard clauses
- Authorization checks
- Business rule enforcement

#### Common Patterns

| Source Pattern | Logic Representation |
|----------------|---------------------|
| `if (!x) throw` | `check(x)` |
| `if (x < y) return` | `check(x >= y)` |
| `if (x && y)` | `check(x && y)` |
| `if (x \|\| y)` | `check(x \|\| y)` |
| `if (!x)` | `check(x)` |

### `action(operation)`

**Purpose:** Represents side effects, state mutations, and operations that change state

**Semantic Meaning:** "This operation modifies state or has external effects"

#### Source Patterns

##### 1. Variable Assignment (State Mutation)

```typescript
this.balance -= amount;
```

**Logic:** `action(deduct_balance)`

```typescript
user.lastLogin = new Date();
```

**Logic:** `action(update_last_login)`

##### 2. Database Operations

```typescript
await this.userRepository.save(user);
```

**Logic:** `action(save_user)`

```typescript
await this.orderRepository.delete(orderId);
```

**Logic:** `action(delete_order)`

```typescript
await this.productRepository.update(id, { stock: newStock });
```

**Logic:** `action(update_stock)`

##### 3. External API Calls

```typescript
await this.emailService.sendWelcomeEmail(user);
```

**Logic:** `action(send_welcome_email)`

```typescript
await this.paymentGateway.charge(amount, card);
```

**Logic:** `action(charge_payment)`

##### 4. Event Emission

```typescript
this.eventEmitter.emit('user.created', user);
```

**Logic:** `action(emit_user_created)`

##### 5. Cache Operations

```typescript
await this.cache.set(key, value);
```

**Logic:** `action(cache_set)`

```typescript
await this.cache.invalidate(key);
```

**Logic:** `action(cache_invalidate)`

##### 6. File System Operations

```typescript
await fs.writeFile(path, content);
```

**Logic:** `action(write_file)`

##### 7. Multiple Actions (Chained)

```typescript
user.balance -= price;
const order = await this.orderRepo.save({ user, item });
await this.emailService.sendConfirmation(order);
```

**Logic:** `action(deduct_balance);action(save_order);action(send_confirmation)`

#### When to Use

- State mutations
- Database writes
- External API calls
- Event emissions
- Cache updates
- File system operations
- Any operation with side effects

#### Naming Conventions

Action names should be:
- **Verb-based:** `save_user`, `send_email`, `update_balance`
- **Descriptive:** Clearly indicate what changes
- **Snake_case:** Use underscores for readability
- **Concise:** Keep under 20 characters when possible

### `return(expression)`

**Purpose:** Represents return statements and their expressions

**Semantic Meaning:** "This is the value or result returned by the method"

#### Source Patterns

##### 1. Simple Value Return

```typescript
return true;
```

**Logic:** `return(true)`

```typescript
return user.balance;
```

**Logic:** `return(user.balance)`

##### 2. Expression Return

```typescript
return user.balance > 0;
```

**Logic:** `return(user.balance > 0)`

```typescript
return items.length === 0;
```

**Logic:** `return(items.length === 0)`

##### 3. Object Return

```typescript
return { valid: true, user };
```

**Logic:** `return({valid:true,user})`

**Note:** Whitespace is minimized for token efficiency.

```typescript
return {
  id: user.id,
  name: user.name,
  email: user.email
};
```

**Logic:** `return({id:user.id,name:user.name,email:user.email})`

##### 4. Method Call Return

```typescript
return this.calculateTotal(items);
```

**Logic:** `return(calculateTotal(items))`

```typescript
return await this.userService.findById(id);
```

**Logic:** `return(userService.findById(id))`

##### 5. Conditional Return (see also `match`)

```typescript
return status === 'active' ? 'allowed' : 'denied';
```

**Logic:** `match(status==='active')?allowed:denied`

**Note:** Ternary operators use `match` instead of `return`.

##### 6. Array/Collection Return

```typescript
return users.filter(u => u.isActive);
```

**Logic:** `return(users.filter(isActive))`

```typescript
return items.map(i => i.price);
```

**Logic:** `return(items.map(price))`

#### When to Use

- Any return statement
- Final result of a method
- Computed values
- Transformed data

#### Simplification Rules

- Remove `await` keyword
- Minimize whitespace
- Simplify lambda expressions
- Preserve operators and comparisons

### `match(pattern)?true_branch:false_branch`

**Purpose:** Represents pattern matching, conditional expressions, and branching logic

**Semantic Meaning:** "Based on this condition, choose between these alternatives"

#### Source Patterns

##### 1. Ternary Operator

```typescript
return status === 'active' ? 'allowed' : 'denied';
```

**Logic:** `match(status==='active')?allowed:denied`

```typescript
const result = user.isAdmin ? 'full_access' : 'limited_access';
```

**Logic:** `match(user.isAdmin)?full_access:limited_access`

##### 2. Switch Statement (Simple)

```typescript
switch (role) {
  case 'admin':
    return 'full';
  case 'user':
    return 'limited';
  default:
    return 'none';
}
```

**Logic:** `match(role)?admin:full,user:limited,default:none`

##### 3. Switch Statement (Complex)

```typescript
switch (status) {
  case 'pending':
    await this.processPending(order);
    break;
  case 'confirmed':
    await this.processConfirmed(order);
    break;
  case 'cancelled':
    await this.processCancelled(order);
    break;
}
```

**Logic:** `match(status)?pending:process_pending,confirmed:process_confirmed,cancelled:process_cancelled`

##### 4. If-Else Chain with Returns

```typescript
if (score >= 90) {
  return 'A';
} else if (score >= 80) {
  return 'B';
} else if (score >= 70) {
  return 'C';
} else {
  return 'F';
}
```

**Logic:** `match(score)?>=90:A,>=80:B,>=70:C,default:F`

##### 5. Nested Ternary

```typescript
return user.isActive 
  ? (user.isAdmin ? 'admin' : 'user')
  : 'inactive';
```

**Logic:** `match(user.isActive)?match(user.isAdmin)?admin:user:inactive`

**Note:** Nested matches are preserved but can become complex.

##### 6. Boolean Match

```typescript
return hasPermission ? allow() : deny();
```

**Logic:** `match(hasPermission)?allow:deny`

#### When to Use

- Ternary operators
- Switch statements
- If-else chains with returns
- Pattern matching logic
- Decision trees

#### Format Rules

- Pattern before `?`
- True branch after `?`
- False branch after `:`
- Multiple cases separated by `,`
- Use `default:` for default case

### `get(source)`

**Purpose:** Represents data retrieval operations without side effects

**Semantic Meaning:** "Retrieve data from this source"

#### Source Patterns

##### 1. Property Access

```typescript
const roles = user.roles;
```

**Logic:** `get(user_roles)`

```typescript
const config = this.config.timeout;
```

**Logic:** `get(config.timeout)`

##### 2. Getter Method Calls

```typescript
const settings = this.getSettings();
```

**Logic:** `get(settings)`

```typescript
const value = await this.configService.get('api_key');
```

**Logic:** `get(config.api_key)`

##### 3. Database Reads

```typescript
const user = await this.userRepo.findById(id);
```

**Logic:** `get(user)`

```typescript
const orders = await this.orderRepo.findByUser(userId);
```

**Logic:** `get(user_orders)`

##### 4. Array/Object Destructuring

```typescript
const { name, email } = user;
```

**Logic:** `get(user.name);get(user.email)`

##### 5. Computed Properties

```typescript
const total = items.reduce((sum, item) => sum + item.price, 0);
```

**Logic:** `get(items_total)`

##### 6. Cache Reads

```typescript
const cached = await this.cache.get(key);
```

**Logic:** `get(cache[key])`

#### When to Use

- Property access
- Getter methods
- Database reads (SELECT queries)
- Cache reads
- Pure data retrieval
- No side effects

#### Distinguishing from `action`

| Operation | Keyword | Reason |
|-----------|---------|--------|
| `user.roles` | `get` | Read-only property access |
| `user.roles = []` | `action` | Modifies state |
| `repo.findById(id)` | `get` | Read-only query |
| `repo.save(user)` | `action` | Modifies database |
| `cache.get(key)` | `get` | Read from cache |
| `cache.set(key, val)` | `action` | Writes to cache |

## Logic Chaining

Multiple logic steps are connected with semicolons (`;`) to represent execution sequence.

### Chaining Rules

1. **Order Matters:** Steps are listed in execution order
2. **Semicolon Separator:** Use `;` between steps
3. **No Trailing Semicolon:** Last step has no semicolon
4. **Maximum Length:** 200 characters total (truncated with `...` if exceeded)

### Example: Complete Method Flow

```typescript
async purchase(user: User, itemId: string): Promise<Order> {
  // Validation checks
  if (!user.isActive) {
    throw new Error("User not active");
  }
  if (this.stock <= 0) {
    throw new Error("Out of stock");
  }
  if (user.balance < this.price) {
    throw new Error("Insufficient funds");
  }
  
  // Get current data
  const item = await this.itemRepo.findById(itemId);
  
  // Perform operations
  user.balance -= this.price;
  this.stock -= 1;
  
  // Save changes
  const order = await this.orderRepo.save({
    user,
    item,
    price: this.price
  });
  
  // Send notification
  await this.emailService.sendConfirmation(order);
  
  // Return result
  return order;
}
```

**Logic:**
```
logic:check(user.isActive);check(stock>0);check(user.balance>=price);get(item);action(deduct_balance);action(decrement_stock);action(save_order);action(send_confirmation);return(order)
```

### Chaining Patterns

#### Pattern 1: Validate → Get → Action → Return

```typescript
async createUser(data: CreateUserDto): Promise<User> {
  if (!data.email) throw new Error("Email required");
  
  const exists = await this.userRepo.findByEmail(data.email);
  if (exists) throw new Error("Email exists");
  
  const hashedPassword = await bcrypt.hash(data.password, 10);
  const user = await this.userRepo.save({ ...data, password: hashedPassword });
  
  return user;
}
```

**Logic:** `check(data.email);get(existing_user);check(!exists);action(hash_password);action(save_user);return(user)`

#### Pattern 2: Get → Check → Match → Action

```typescript
async processOrder(orderId: string): Promise<void> {
  const order = await this.orderRepo.findById(orderId);
  
  if (!order) throw new Error("Order not found");
  
  switch (order.status) {
    case 'pending':
      await this.processPending(order);
      break;
    case 'confirmed':
      await this.processConfirmed(order);
      break;
  }
}
```

**Logic:** `get(order);check(order);match(order.status)?pending:process_pending,confirmed:process_confirmed`

#### Pattern 3: Check → Action → Action → Return

```typescript
async withdraw(amount: number): Promise<boolean> {
  if (this.balance < amount) {
    return false;
  }
  
  this.balance -= amount;
  await this.transactionRepo.save({ type: 'withdrawal', amount });
  
  return true;
}
```

**Logic:** `check(balance>=amount);action(deduct_balance);action(save_transaction);return(true)`

## Complex Examples

### Example 1: Authentication Flow

```typescript
async login(email: string, password: string): Promise<{ token: string }> {
  // Find user
  const user = await this.userRepo.findByEmail(email);
  if (!user) {
    throw new UnauthorizedException("Invalid credentials");
  }
  
  // Check password
  const isValid = await bcrypt.compare(password, user.password);
  if (!isValid) {
    throw new UnauthorizedException("Invalid credentials");
  }
  
  // Check if active
  if (!user.isActive) {
    throw new ForbiddenException("Account disabled");
  }
  
  // Update last login
  user.lastLogin = new Date();
  await this.userRepo.save(user);
  
  // Create session
  const token = await this.jwtService.sign({ userId: user.id });
  
  return { token };
}
```

**Logic:**
```
logic:get(user);check(user);check(password_valid);check(user.isActive);action(update_last_login);action(save_user);action(create_token);return({token})
```

### Example 2: E-Commerce Checkout

```typescript
async checkout(cart: Cart, paymentMethod: PaymentMethod): Promise<Order> {
  // Validate cart
  if (cart.items.length === 0) {
    throw new BadRequestException("Cart is empty");
  }
  
  // Calculate total
  const total = cart.items.reduce((sum, item) => sum + item.price, 0);
  
  // Check balance
  const user = await this.userRepo.findById(cart.userId);
  if (user.balance < total) {
    throw new BadRequestException("Insufficient funds");
  }
  
  // Reserve stock
  for (const item of cart.items) {
    const product = await this.productRepo.findById(item.productId);
    if (product.stock < item.quantity) {
      throw new BadRequestException(`Insufficient stock for ${product.name}`);
    }
    product.stock -= item.quantity;
    await this.productRepo.save(product);
  }
  
  // Process payment
  const payment = await this.paymentService.charge(paymentMethod, total);
  if (!payment.success) {
    // Rollback stock
    await this.rollbackStock(cart.items);
    throw new BadRequestException("Payment failed");
  }
  
  // Create order
  const order = await this.orderRepo.save({
    userId: cart.userId,
    items: cart.items,
    total,
    status: 'confirmed'
  });
  
  // Send confirmation
  await this.emailService.sendOrderConfirmation(order);
  
  return order;
}
```

**Logic:**
```
logic:check(cart.items.length>0);get(total);get(user);check(user.balance>=total);action(reserve_stock);action(charge_payment);match(payment.success)?action(create_order);action(send_confirmation);return(order):action(rollback_stock)
```

**Note:** This is truncated at 200 characters in actual output.

### Example 3: Access Control

```typescript
async canActivate(context: ExecutionContext): Promise<boolean> {
  // Get request
  const request = context.switchToHttp().getRequest();
  
  // Get user from request
  const user = request.user;
  if (!user) {
    return false;
  }
  
  // Get required roles from metadata
  const requiredRoles = this.reflector.get<string[]>('roles', context.getHandler());
  if (!requiredRoles || requiredRoles.length === 0) {
    return true;
  }
  
  // Check if user has required role
  const hasRole = requiredRoles.some(role => user.roles.includes(role));
  
  return hasRole;
}
```

**Logic:**
```
logic:get(request);get(user);check(user);get(required_roles);match(required_roles.length===0)?return(true):check(has_role);return(has_role)
```

### Example 4: Data Transformation Pipeline

```typescript
async processData(rawData: RawData[]): Promise<ProcessedData[]> {
  // Filter invalid entries
  const validData = rawData.filter(d => d.isValid);
  
  // Transform data
  const transformed = validData.map(d => ({
    id: d.id,
    value: d.value * 2,
    timestamp: new Date()
  }));
  
  // Save to database
  await this.dataRepo.saveMany(transformed);
  
  // Invalidate cache
  await this.cache.invalidate('processed_data');
  
  // Emit event
  this.eventEmitter.emit('data.processed', transformed.length);
  
  return transformed;
}
```

**Logic:**
```
logic:get(valid_data);get(transformed);action(save_data);action(invalidate_cache);action(emit_event);return(transformed)
```

## Extraction Rules

### Heuristics for Classification

YCG uses the following heuristics to classify operations:

#### 1. Side Effect Detection

An operation has side effects if it:
- Assigns to a non-local variable
- Calls a method that mutates state
- Performs I/O operations
- Modifies database state
- Sends network requests

#### 2. Pure Function Detection

An operation is pure (use `get`) if it:
- Only reads data
- Has no side effects
- Returns the same result for same inputs
- Doesn't modify external state

#### 3. Control Flow Detection

Control flow patterns:
- `if` with `throw` → `check`
- `if` with early `return` → `check`
- Ternary operator → `match`
- `switch` statement → `match`

### Extraction Order

Logic steps are extracted in execution order:

1. **Preconditions:** All `check` statements at method start
2. **Data Retrieval:** `get` operations
3. **Business Logic:** `action` and `match` operations
4. **Return:** Final `return` statement

### Simplification Rules

1. **Remove Noise:**
   - Remove `await` keyword
   - Remove `async` keyword
   - Minimize whitespace
   - Remove type annotations

2. **Preserve Semantics:**
   - Keep logical operators (`&&`, `||`, `!`)
   - Keep comparison operators (`===`, `!==`, `<`, `>`, etc.)
   - Keep arithmetic operators in conditions

3. **Abbreviate Names:**
   - `this.userRepository.save(user)` → `save_user`
   - `await this.emailService.sendWelcomeEmail(user)` → `send_welcome_email`
   - `const roles = user.roles` → `get(user_roles)`

## Edge Cases

### Truncation

Logic representations are limited to 200 characters. If exceeded, the output is truncated with `...`:

```
logic:check(user.isActive);check(stock>0);check(balance>=price);get(item);action(deduct_balance);action(decrement_stock);action(save_order);action(send_confirmation);action(update_inventory);action(log_transaction)...
```

### No Extractable Logic

If a method has no extractable logic (e.g., simple getter), the logic field is omitted:

```yaml
# Method with no logic
- UserService_getName_a1b2|getName():str|method

# Not:
- UserService_getName_a1b2|getName():str|method|logic:
```

### Complex Nested Logic

Deeply nested logic may be simplified:

```typescript
if (user.isActive) {
  if (user.isAdmin) {
    if (user.hasPermission('write')) {
      // ...
    }
  }
}
```

**Logic:** `check(user.isActive && user.isAdmin && user.hasPermission('write'))`

### Async/Await

`async` and `await` keywords are removed as they don't affect logic semantics:

```typescript
const user = await this.userRepo.findById(id);
```

**Logic:** `get(user)`

### Try-Catch Blocks

Error handling is generally omitted unless it contains business logic:

```typescript
try {
  await this.process(data);
} catch (error) {
  this.logger.error(error);
  throw error;
}
```

**Logic:** `action(process)`

**Note:** The catch block is omitted as it's just logging.

### Loops

Loops are simplified to their essential operation:

```typescript
for (const item of items) {
  await this.processItem(item);
}
```

**Logic:** `action(process_items)`

```typescript
items.forEach(item => this.validate(item));
```

**Logic:** `action(validate_items)`

## Best Practices

### For Developers

1. **Write Clear Code:** Logic extraction works best with clear, well-structured code
2. **Use Guard Clauses:** Early returns and guard clauses are extracted cleanly
3. **Avoid Deep Nesting:** Flat logic is easier to extract and understand
4. **Meaningful Names:** Use descriptive method and variable names

### For Analysts

1. **Focus on Patterns:** Look for overall patterns, not individual keywords
2. **Verify Critical Logic:** For security-critical code, verify logic against source
3. **Understand Limitations:** Logic extraction is best-effort, not perfect
4. **Use Selectively:** Apply Level 2 only where logic matters

### For Tool Users

1. **Start Simple:** Begin with Level 0 or 1, upgrade to Level 2 selectively
2. **Combine with Source:** Use logic as a guide, refer to source for details
3. **Monitor Token Usage:** Level 2 adds 30-40% tokens, use wisely
4. **Provide Feedback:** Report unexpected logic extractions to improve heuristics

## Further Reading

- [GRANULARITY_GUIDE.md](GRANULARITY_GUIDE.md) - Complete granularity guide
- [README.md](README.md) - Main documentation
- `.kiro/specs/adhoc-granularity-levels/design.md` - Technical design
- `.kiro/specs/adhoc-granularity-levels/requirements.md` - Requirements specification
