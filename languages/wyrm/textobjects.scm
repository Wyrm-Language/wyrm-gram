(function_definition
  body: (block
    ":" (_)* @function.inside)) @function.around

(coroutine_definition
  body: (block
    ":" (_)* @function.inside)) @function.around

(class_definition
  body: (block
    ":" (_)* @class.inside)) @class.around

(comment) @comment.around
