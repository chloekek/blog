/// Document that there must be a current OpenGL context.
#[macro_export]
macro_rules! doc_safety_opengl
{
    () => { "# Safety\n\nThere must be a current OpenGL context." };
}
