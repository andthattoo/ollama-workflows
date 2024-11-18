use ollama_workflows::Workflow;

#[test]
fn test_task_mutable() {
    let mut workflow = Workflow::new_from_json("./tests/test_workflows/search.json").unwrap();

    let task_id = "E";
    let task = workflow.get_tasks_by_id_mut(task_id).unwrap();
    assert_eq!(task.id, task_id);

    assert_eq!(task.messages.len(), 1);
    task.append_assistant_message("This is your response.");
    task.append_user_message("Thank you.");
    assert_eq!(task.messages.len(), 3);
}
