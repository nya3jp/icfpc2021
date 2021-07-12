import React, {useState} from 'react';

import Button from '@material-ui/core/Button';
import Chip from '@material-ui/core/Chip';
import DirectionsRunIcon from '@material-ui/icons/DirectionsRun';
import EditIcon from '@material-ui/icons/Edit';
import SendIcon from '@material-ui/icons/Send';
import Snackbar from '@material-ui/core/Snackbar';

import {Model} from './model';
import {Problem} from './types';
import {useHistory} from 'react-router-dom';

export const BonusChip = ({bonus, text}: {bonus: string, text: string}) => {
    let fg = "";
    let bg = "";
    if (bonus === "GLOBALIST") {
        fg = "#000";
        bg = "#ffeb3b";
    } else if (bonus === "BREAK_A_LEG") {
        fg = "#fff";
        bg = "#26a69a";
    } else if (bonus === "WALLHACK") {
        fg = "#000";
        bg = "#ff9100";
    } else if (bonus === "SUPERFLEX") {
        fg = "#fff";
        bg = "#ff1744";
    }

    return <Chip color='primary' style={{color: fg, backgroundColor: bg}} label={text} />
};

export interface EditButtonProps {
    problemID: number;
    solutionID: number;
    text?: string;
};

export const EditButton = (props: EditButtonProps) => {
    const {problemID, solutionID, text = 'エディタで編集'} = props;

    return (
        <Button color="secondary" href={`https://nya3jp.github.io/icfpc2021/fcc7938b3c545e6ff51b101ea86f548b/#?problem_id=${problemID}&base_solution_id=${solutionID}`} endIcon={<EditIcon />} variant="contained">{text}</Button>
    );
};

export interface OfficialSubmitButtonProps {
    model: Model;
    solutionID: number;
    text?: string;
};

export const OfficialSubmitButton = (props: OfficialSubmitButtonProps) => {
    const {model, solutionID, text = '公式にSubmit'} = props;

    const [sending, setSending] = useState<boolean>(false);
    const [message, setMessage] = useState<string>("");
    const [openTimedMessage, setOpenTimedMessage] = useState<boolean>(false);
    const [timedMessage, setTimedMessage] = useState<string>("");

    const handleSend = async () => {
        setMessage("Sending the solution...");
        setSending(true);
        let resp = await model.submitSolution(solutionID);
        setSending(false);
        setTimedMessage(resp);
        setOpenTimedMessage(true);
    };
    const handleClose = () => {
        setOpenTimedMessage(false);
    };

    return (
        <React.Fragment>
            <Button disabled={sending} color="primary" onClick={handleSend} endIcon={<SendIcon />} variant="contained">{text}</Button>
            <Snackbar open={sending} message={message} />
            <Snackbar open={openTimedMessage} autoHideDuration={3000} onClose={handleClose} message={timedMessage} />
        </React.Fragment>
    );
};

export interface RunSolverButtonProps {
    model: Model;
    problem: Problem;
    bonus: string;
    text?: string;
};

export const RunSolverButton = (props: RunSolverButtonProps) => {
    const {model, problem, bonus, text = 'ソルバを叩く'} = props;

    const [sending, setSending] = useState<boolean>(false);
    const [message, setMessage] = useState<string>("");
    const history = useHistory();

    const handleSend = async () => {
        setMessage("Triggering the solver...");
        setSending(true);
        try {
            const taskID = await model.triggerSolver(problem.problem_id, bonus, problem.minimal_dislike, 10 /* sec */, 60 /* sec */);
            history.push(`/tasks/${taskID}`);
        } finally {
            setSending(false);
        }
    };

    return (
        <React.Fragment>
            <Button disabled={sending} color="primary" onClick={handleSend} endIcon={<DirectionsRunIcon />} variant="contained">{text}</Button>
            <Snackbar open={sending} message={message} />
        </React.Fragment>
    );
};
